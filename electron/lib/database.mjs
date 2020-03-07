import Realm from 'realm'
import blake3 from 'blake3'
import buffer from 'buffer'
import crypto from 'crypto'
import uuid from 'uuid/v4'

const SCHEMAS = []

const MessageSchema = {
  name: 'Message',
  primaryKey: 'id',
  properties: {
    chatroom: { type: 'linkingObjects', objectType: 'Chatroom', property: 'messages' },
    content: 'Blob?',
    id: 'string',
    participants: 'string[]',
    sender: 'string',
    time: { type: 'date', indexed: true },
    read: { type: 'bool', default: true }
  }
}
SCHEMAS.push(MessageSchema)

const ChatroomSchema = {
  name: 'Chatroom',
  primaryKey: 'id',
  properties: {
    id: 'string',
    members: 'string[]',
    messages: 'Message[]',
    name: 'string?'
  }
}
SCHEMAS.push(ChatroomSchema)

const VcardSchema = {
  name: 'Vcard',
  primaryKey: 'id',
  properties: {
    avatar: 'Blob?',
    id: 'string',
    name: { type: 'string', default: '' },
    time_updated: 'date?'
  }
}
SCHEMAS.push(VcardSchema)

const ProfileSchema = {
  name: 'Profile',
  primaryKey: 'name',
  properties: {
    name: 'string',
    value: 'data?'
  }
}
SCHEMAS.push(ProfileSchema)

const PeerSchema = {
  name: 'Peer',
  primaryKey: 'id',
  properties: {
    id: 'string',
    role: {type: 'int', default: 0 }
  }
}
SCHEMAS.push(PeerSchema)

export class Blob {
  constructor (content, mime) {
    this.content = content
    this.mime = mime
    this.id = uuid()
  }

  static createText (content) {
    return new Blob(buffer.Buffer.from(content))
  }
}
SCHEMAS.push({
  name: 'Blob',
  primaryKey: 'id',
  properties: {
    content: 'data?',
    id: 'string',
    mime: 'string?'
  }
})

// Fields in the Profile table
const PROFILE_CERTIFICATE = 'certificate'
const PROFILE_ID = 'id'
const PROFILE_KEYPAIR = 'keypair'

export class Database {
  static async open (path) {
    const db = new Database()
    db.realm = await Realm.open({
      schema: SCHEMAS,
      path: path
    })
    return db
  }

  close () {
    this.realm.close()
  }

  /**
   * Must run in an empty database.
   */
  createAccount (vcard) {
    if (!this.realm.empty) {
      console.warn('Creating account in a non-empty database')
    }

    const bundle = newCertificate()
    const id = blake3.hash(bundle.certificate).toString('hex')
    vcard.id = id

    this.realm.write(() => {
      this.realm.create(VcardSchema.name, vcard)
      this.realm.create(ProfileSchema.name, {
        name: PROFILE_CERTIFICATE,
        value: bundle.certificate
      })
      this.realm.create(ProfileSchema.name, {
        name: PROFILE_KEYPAIR,
        value: bundle.keypair
      })
      this.realm.create(ProfileSchema.name, {
        name: PROFILE_ID,
        value: buffer.Buffer.from(id)
      })
    })
  }

  putPeer (peer) {
    this.realm.write(() => {
      this.realm.create(PeerSchema.name, peer, 'all')
    })
  }

  putVcard (vcard) {
    this.realm.write(() => {
      this.realm.create(VcardSchema.name, vcard, 'all')
    })
  }

  putChatroom (chatroom) {
    this.realm.write(() => {
      chatroom.members = normalizeChatroomMembers(chatroom.members)
      this.realm.create(ChatroomSchema.name, chatroom, 'all')
    })
  }

  /**
   * Gets a chatroom by its members. A new chartoom will be created if the chatroom does not exist.
   * @param {string[]} members
   */
  getChatroom (members) {
    let result
    this.realm.write(() => {
      result = chatroomByMembers(this.realm, normalizeChatroomMembers(members))
    })
    return result
  }

  putMessage (message) {
    message.participants = normalizeChatroomMembers(message.participants)
    this.realm.write(() => {
      const oldMessageObj = this.realm.objectForPrimaryKey(MessageSchema.name, message.id)
      if (!oldMessageObj) {
        const newMessageObj = this.realm.create(MessageSchema.name, message)
        chatroomByMembers(this.realm, message.participants).messages.push(newMessageObj)
      } else if (oldMessageObj.participants === message.participants) {
        this.realm.create(MessageSchema.name, message, 'all')
      } else {
        throw new Error('The message to overwrite has different participants')
      }
    })
  }
}

// TODO: Call the Rust function
function newCertificate () {
  return {
    certificate: crypto.randomBytes(64).buffer,
    keypair: crypto.randomBytes(32).buffer
  }
}

/**
 * @param {string[]} normalizedMembers
 * @returns {string}
 */
function chatroomId (normalizedMembers) {
  const hasher = blake3.createHash()
  for (const m of normalizedMembers) {
    hasher.update(m)
  }
  return hasher.digest().toString('hex')
}

/**
 * Must be run inside a transaction
 * @param {Realm} realm
 * @param {string[]} normalizedMembers
 */
function chatroomByMembers (realm, normalizedMembers) {
  const id = chatroomId(normalizedMembers)
  let result = realm.objectForPrimaryKey(ChatroomSchema.name, id)
  if (!result) {
    result = realm.create(ChatroomSchema.name, {
      id: id,
      members: normalizedMembers
    })
  }
  return result
}

/**
 * @param {string[]} members
 */
function normalizeChatroomMembers (members) {
  const normalized = []
  for (const i of members) {
    normalized.push(i.toLowerCase())
  }
  return normalized.sort()
}

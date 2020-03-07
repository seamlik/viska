import {
  Blob,
  Database
} from '../lib/database'

import crypto from 'crypto'
import faker from 'faker'
import fs from 'fs'
import mime from 'mime'
import random from 'random'
import yargs from 'yargs'

async function main () {
  const realmLocation = yargs.argv._[0]
  try {
    await fs.promises.unlink(realmLocation)
  } catch (err) {
    // Ignore if the file does not exists
  }
  const db = await Database.open(realmLocation)

  const accountVcard = randomVcard()
  accountVcard.avatar = await readProjectLogoAsAvatar()
  db.createAccount(accountVcard)
  console.debug(`Created account with ID ${accountVcard.id}`)

  console.debug('Generating blacklsit…')
  for (let i = 0; i < 10; ++i) {
    const peer = randomPeer()
    peer.role = -1
    db.putPeer(peer)
    db.putVcard(randomVcard(peer.id))
  }

  console.debug('Generating whitelsit…')
  const whitelist = []
  for (let i = 0; i < 10; ++i) {
    const peer = randomPeer()
    db.putPeer(peer)
    db.putVcard(randomVcard(peer.id))
    whitelist.push(peer)
  }

  // Chatroom and messages
  for (let i = 0; i < 5; ++i) {
    const members = [whitelist[i].id, whitelist[i + 5].id]
    const chatroom = db.getChatroom(members)
    console.debug(`Generating messages for chatroom ${chatroom.id}…`)
    for (let j = 0; j < 10; ++j) {
      db.putMessage(randomMessage(members))
    }
  }

  console.debug('Closing database…')
  db.close()
}

/**
 * @param {string[]} participants
 */
function randomMessage (participants) {
  const content = Blob.createText(faker.lorem.paragraph())
  return {
    content: content,
    id: content.id,
    participants: participants,
    sender: participants[random.int(0, participants.length - 1)],
    time: faker.date.recent()
  }
}

async function readProjectLogoAsAvatar () {
  return new Blob(
    await fs.promises.readFile('logo.svg'),
    mime.getType('svg')
  )
}

function randomPeer () {
  return {
    id: crypto.randomBytes(32).toString('hex')
  }
}

/**
 * @param {string} id
 */
function randomVcard (id) {
  return {
    id: id,
    name: faker.name.findName()
  }
}

main()

package viska.database

import android.util.Log
import com.couchbase.lite.Database
import org.apache.commons.codec.binary.Hex
import org.bson.BsonBinary

val LOG_TAG = "viska.database"

fun open() = Database("main")

fun Database.initialize() {
  TODO()
}

fun Database.createDemoProfile(): Unit = TODO()

fun Database.createNewProfile() {
  val bundle = viska.pki.Module.new_certificate()!!
  val certificate = bundle.asDocument().getBinary("certificate").data
  val keypair = bundle.asDocument().getBinary("keypair").data

  val accountId = viska.pki.Module.hash(BsonBinary(certificate))?.asBinary()?.data
  Log.i(LOG_TAG, "Generated account ${Hex.encodeHexString(accountId, false)}")
  Profile.fromPkiBundle(this, certificate, keypair).save()
}

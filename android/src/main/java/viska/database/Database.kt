package viska.database

import android.content.Context
import android.util.Log
import androidx.core.content.edit
import androidx.preference.PreferenceManager
import com.couchbase.lite.Database
import java.nio.file.Files
import org.bson.BsonBinary
import viska.database.Module.display_id
import viska.database.Module.hash

const val LOG_TAG = "viska.database"

const val MIME_ID = "application/viska-id"

const val MIME_TEXT = "text/*"

fun Database.initialize() {
  TODO()
}

fun Database.createDemoProfile(): Unit = TODO()

fun Context.createNewProfile() {
  val bundle = viska.pki.Module.new_certificate()!!
  val certificate = bundle.asDocument().getBinary("certificate").data
  val key = bundle.asDocument().getBinary("key").data

  val accountId = hash(BsonBinary(certificate))?.asBinary()?.data
  val displayAccountId = display_id(BsonBinary(accountId))!!.asString().value
  Log.i(LOG_TAG, "Generated account $displayAccountId")

  val profileDir = filesDir.toPath().resolve("account").resolve(displayAccountId)
  Files.createDirectories(profileDir)
  Files.write(profileDir.resolve("certificate.der"), certificate)
  Files.write(profileDir.resolve("key.der"), key)

  PreferenceManager.getDefaultSharedPreferences(this).edit(commit = true) {
    putString("active-account", displayAccountId)
  }
}

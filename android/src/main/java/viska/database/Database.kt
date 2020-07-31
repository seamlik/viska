package viska.database

import android.content.Context
import android.util.Log
import androidx.core.content.edit
import androidx.preference.PreferenceManager
import com.couchbase.lite.Database
import java.nio.file.Files
import org.bson.BsonBinary

const val LOG_TAG = "viska.database"

const val MIME_ACCOUNT_ID = "application/viska-account-id"

fun Database.initialize() {
  TODO()
}

fun Database.createDemoProfile(): Unit = TODO()

fun Context.createNewProfile() {
  val bundle = viska.pki.Module.new_certificate()!!
  val certificate = bundle.asDocument().getBinary("certificate").data
  val key = bundle.asDocument().getBinary("key").data

  val accountId = viska.pki.Module.hash(BsonBinary(certificate))?.asBinary()?.data
  val displayAccountId = viska.pki.Module.display_id(BsonBinary(accountId))!!.asString().value
  Log.i(LOG_TAG, "Generated account $displayAccountId")

  val profileDir = filesDir.toPath().resolve("account").resolve(displayAccountId)
  Files.createDirectories(profileDir)
  Files.write(profileDir.resolve("certificate.der"), certificate)
  Files.write(profileDir.resolve("key.der"), key)

  PreferenceManager.getDefaultSharedPreferences(this).edit(commit = true) {
    putString("active-account", displayAccountId)
  }
}

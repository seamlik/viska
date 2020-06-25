package viska.database

import com.couchbase.lite.Database
import com.couchbase.lite.MutableDocument

fun open() = Database("main")

fun Database.initialize() {
  TODO()
}

fun Database.createDemoProfile(): Unit = TODO()

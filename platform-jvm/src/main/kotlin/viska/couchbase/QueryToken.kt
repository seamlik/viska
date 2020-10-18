package viska.couchbase

import com.couchbase.lite.Database
import com.couchbase.lite.ListenerToken
import com.couchbase.lite.Query

class DocumentChangeToken(private val token: ListenerToken, private val database: Database) :
    AutoCloseable {
  override fun close() {
    database.removeChangeListener(token)
  }
}

class LiveQueryToken(private val token: ListenerToken, private val query: Query) : AutoCloseable {
  override fun close() {
    query.removeChangeListener(token)
  }
}

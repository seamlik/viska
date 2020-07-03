package viska.android

import android.util.Log
import androidx.recyclerview.widget.DiffUtil
import androidx.recyclerview.widget.ListAdapter
import androidx.recyclerview.widget.RecyclerView
import com.couchbase.lite.ListenerToken
import com.couchbase.lite.Query
import com.couchbase.lite.Result

abstract class CouchbaseLiveQueryListAdapter<VH : RecyclerView.ViewHolder>(
    private val query: Query, differ: DiffUtil.ItemCallback<Result>
) : ListAdapter<Result, VH>(differ) {

  private val token: ListenerToken

  init {
    token =
        query.addChangeListener { change ->
          change.error?.apply {
            Log.e(CouchbaseLiveQueryListAdapter::class.qualifiedName, null, this)
          }
          submitList(change.results.toList())
        }
    query.execute()
  }

  fun unsubscribe() {
    query.removeChangeListener(token)
  }
}

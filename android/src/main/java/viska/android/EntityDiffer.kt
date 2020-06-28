package viska.android

import androidx.recyclerview.widget.DiffUtil
import com.couchbase.lite.Result
import viska.database.Entity

class EntityDiffer<T : Entity>(private val factory: (Result) -> T) :
    DiffUtil.ItemCallback<Result>() {
  override fun areItemsTheSame(oldItem: Result, newItem: Result) =
      factory(oldItem).documentId == factory(newItem).documentId

  override fun areContentsTheSame(oldItem: Result, newItem: Result) =
      factory(oldItem) == factory(newItem)
}

package viska.android

import androidx.lifecycle.MutableLiveData

object GlobalState {
  val creatingAccount = MutableLiveData<Boolean>(false)
}

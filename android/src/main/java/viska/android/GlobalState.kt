package viska.android

import kotlinx.coroutines.flow.MutableStateFlow

object GlobalState {
  val creatingAccount = MutableStateFlow(false)
}

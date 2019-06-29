package viska.android.main;

import androidx.lifecycle.MutableLiveData;

class ViewModel extends androidx.lifecycle.ViewModel {

  final MutableLiveData<Screen> screens = new MutableLiveData<>();

  ViewModel() {
    screens.setValue(Screen.CHATROOMS);
  }
}

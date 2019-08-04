package viska.android;

import androidx.lifecycle.MutableLiveData;

public class MainViewModel extends androidx.lifecycle.ViewModel {

  final MutableLiveData<MainScreen> screen = new MutableLiveData<>();

  MainViewModel() {
    screen.setValue(MainScreen.CHATROOMS);
  }
}

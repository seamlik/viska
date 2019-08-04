package viska.android;

import androidx.lifecycle.MutableLiveData;

public class ApplicationViewModel extends androidx.lifecycle.ViewModel {

  public final MutableLiveData<Boolean> creatingAccount = new MutableLiveData<>();

  public ApplicationViewModel() {
    creatingAccount.setValue(false);
  }
}

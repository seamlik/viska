package chat.viska.android.main;

import android.content.Context;
import android.util.AttributeSet;
import android.view.LayoutInflater;
import android.view.ViewStub;
import androidx.annotation.LayoutRes;
import androidx.coordinatorlayout.widget.CoordinatorLayout;
import chat.viska.R;
import com.google.android.material.appbar.MaterialToolbar;

public abstract class ContentView extends CoordinatorLayout {
  private MaterialToolbar actionBar;

  public ContentView(Context context, AttributeSet attrs, int defStyleAttr) {
    super(context, attrs, defStyleAttr);
    LayoutInflater.from(getContext()).inflate(R.layout.main_screen, this, true);
    actionBar = findViewById(R.id.action_bar);
  }

  public void inflateStub(@LayoutRes final int layout) {
    final ViewStub stub = findViewById(R.id.container_content);
    stub.setLayoutResource(layout);
    stub.inflate();
  }

  public MaterialToolbar getActionBar() {
    return actionBar;
  }
}

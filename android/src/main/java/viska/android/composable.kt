package viska.android

import androidx.compose.foundation.Box
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Stack
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material.ListItem
import androidx.compose.material.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.unit.dp

@Composable
fun DrawerNavigationItem(
    selected: Boolean,
    icon: @Composable () -> Unit,
    text: @Composable () -> Unit,
    onClick: () -> Unit,
) {
  Stack {
    if (selected) {
      Box(
          modifier = Modifier.width(4.dp).height(24.dp).align(Alignment.CenterStart),
          shape = RectangleShape,
          backgroundColor = MaterialTheme.colors.onBackground,
      )
    }
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        icon = icon,
        text = text,
    )
  }
}

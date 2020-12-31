package viska.android

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
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
  Box {
    if (selected) {
      Box(
          modifier =
              Modifier.width(4.dp)
                  .height(24.dp)
                  .background(MaterialTheme.colors.onBackground, RectangleShape)
                  .align(Alignment.CenterStart),
      )
    }
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        icon = icon,
        text = text,
    )
  }
}

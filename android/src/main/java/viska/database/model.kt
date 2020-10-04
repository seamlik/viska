package viska.database

import java.time.Instant

data class Chatroom(
    val name: String,
    val members: Set<String>,
    val latestMessage: Database.Message? = null,
    val timeUpdated: Instant,
    val chatroomId: String,
)

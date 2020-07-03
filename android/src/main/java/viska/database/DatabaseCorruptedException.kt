package viska.database

import java.lang.RuntimeException

class DatabaseCorruptedException(msg: String?) : RuntimeException(msg)

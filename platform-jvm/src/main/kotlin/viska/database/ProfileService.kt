package viska.database

interface ProfileService {
  val accountId: String
  fun createProfile(mock: Boolean)
  val profileConfig: ProfileConfig
}

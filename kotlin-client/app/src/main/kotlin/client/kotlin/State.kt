package client.kotlin

import kotlinx.serialization.Serializable
import java.net.Socket

enum class Teams {
    Red, Black
}

@Serializable
data class Player(val id: Int, val name: String, val ready: Boolean, val team: Teams, var health: Int);

data class PlayerState(val socket: Socket, var player: Player)

@Serializable
data class GameState(val team_red: HashMap<String, Player>, val team_black: HashMap<String, Player>, val current_turn: Teams)


object MainCharacter {
    lateinit var state: PlayerState;
}
package client.kotlin
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import tornadofx.*

class CombatController: Controller() {
    var model: CombatModel
    val enemyNames: ArrayList<String>
        get() {
            return model.enemyNames
        }
    val enemyStatuses: String
        get() {
            var result = "Enemies:\n"
            if (MainCharacter.state.player.team == Teams.Black) {
                model.gameState.team_red.forEach {
                    result += it.key + " : " + it.value.health + "\n"
                }
            } else {
                model.gameState.team_black.forEach {
                    result += it.key + " : " + it.value.health + "\n"
                }
            }
            return result
        }

    val turn: Teams
        get() = model.gameState.current_turn

    val allyStatuses: String
        get() {
            var result = "Allies:\n"
            if (MainCharacter.state.player.team == Teams.Black) {
                model.gameState.team_black.forEach {
                    result += it.key + " : " + it.value.health + "\n"
                }
            } else {
                model.gameState.team_red.forEach {
                    result += it.key + " : " + it.value.health + "\n"
                }
            }
            return result
        }

    init {
        println("Combat controller initializing")
        model = update()
        println("Combat controller initialized")
    }
    
    private fun update(): CombatModel {
        val line = MainCharacter.state.socket.getInputStream().bufferedReader().readLine()
        val gameState: GameState = Json.decodeFromString(line);
        val enemyNames = ArrayList<String>()
        if (MainCharacter.state.player.team == Teams.Black) {
            gameState.team_red.forEach {
                enemyNames.add(it.key)
            }
        } else {
            gameState.team_black.forEach {
                enemyNames.add(it.key)
            }
        }
        return CombatModel(gameState, enemyNames)
    }
    
    fun triggerUpdate() {
        model = update()
    }
    
    fun attack(name: String) {
        println("Attacking...")
       val writer = MainCharacter.state.socket.getOutputStream()
        println("Sending attack...")
        writer.write("attack".toByteArray())
        println("Sending name...")
        writer.write(name.toByteArray())
    }
    
    fun heal(name: String) {

    }
}
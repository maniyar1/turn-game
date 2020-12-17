package client.kotlin
import javafx.beans.property.SimpleStringProperty
import tornadofx.*
import java.net.Socket

class ConnectController: Controller() {
    
    fun connect(ip: String, team: Teams, name: String, status: SimpleStringProperty) {
        val client = Socket(ip, 1370)
        val writer = client.getOutputStream();
        val reader = client.getInputStream().bufferedReader();
        status.value = reader.readLine()
        writer.write(team.toString().toLowerCase().toByteArray()) // Team
        status.value = reader.readLine()
        writer.write(name.toByteArray()) // Name
        status.value = reader.readLine()
        val player = Player(0, name, false, team, 100);
        MainCharacter.state = PlayerState(client, player)
    }

    fun ready(status: SimpleStringProperty) {
        MainCharacter.state.socket.getOutputStream().write("ready".toByteArray()) // ready
        println("ready")
    }
}
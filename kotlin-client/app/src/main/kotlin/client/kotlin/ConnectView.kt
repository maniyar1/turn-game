package client.kotlin
import javafx.beans.property.SimpleObjectProperty
import javafx.beans.property.SimpleStringProperty
import javafx.collections.FXCollections
import javafx.scene.control.Alert
import tornadofx.*

class ConnectView: View() {
    private val controller: ConnectController by inject()
    private val teams = FXCollections.observableArrayList(Teams.Red, Teams.Black)

    private val ip = SimpleStringProperty()
    private val name = SimpleStringProperty()
    private val selectedTeam = SimpleObjectProperty<Teams>();

    private val status = SimpleStringProperty()

    override val root = form {
        fieldset {
            field("Ip") {
                textfield(ip)
            }
            field("Team") {
                combobox(selectedTeam, teams)
            }
            field("Name") {
                textfield(name)
            }
            button("Connect") {
                action {
                    run {
                        controller.connect(ip.value, selectedTeam.value, name.value, status)
                        alert(Alert.AlertType.INFORMATION, "Ready? (Make sure all players have connected)")
                        controller.ready(status)
                        replaceWith<CombatView>()
                    }
                }
            }
            field("Status") {
                label(status)
            }
        }
    }
}

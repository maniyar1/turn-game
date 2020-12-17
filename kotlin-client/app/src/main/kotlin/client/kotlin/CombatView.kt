package client.kotlin
import javafx.collections.FXCollections
import javafx.scene.control.Button
import javafx.scene.control.ComboBox
import javafx.scene.control.Label
import javafx.scene.text.Text
import tornadofx.*

class CombatView: View() {
    private val controller: CombatController = CombatController()
    private val enemyNameList = FXCollections.observableArrayList<String>()
    private val enemies: ComboBox<String> = ComboBox(enemyNameList)
    private val enemyStatuses: Text = Text()
    private val allyStatuses: Text = Text()
    private var attack = button()
    private var heal = button()
    private var turnLabel = Label()

    override val root = vbox {
        this.add(enemyStatuses)
        this.add(allyStatuses)
        this.add(enemies)
        this.add(turnLabel)
        hbox {
            attack = button("Attack") {
                action {
                    run {
                        controller.attack(enemies.value)
                        setButtonsDisable(true)
                        controller.triggerUpdate()
                        updateView() // Update for this turn
                        runAsync {
                            controller.triggerUpdate() // Wait for next turn's update..
                            println("done")
                        }.setOnSucceeded {
                            updateView()
                        }
                    }
                }
            }
            heal = button("Heal") {
                action {
                    run {
                        controller.heal(enemies.value)
                    }
                }
            }
        }
    }
    
    fun updateView() {
        enemyStatuses.text = controller.enemyStatuses
        allyStatuses.text = controller.allyStatuses
        updateViewTurn()
    }

    fun updateViewTurn() {
        if (MainCharacter.state.player.team != controller.turn) {
            println("Updating label (enemy turn)")
            turnLabel.text = "Enemy turn, please wait..."
            println("Updating buttons")
            setButtonsDisable(true)
            runAsync {
                println("Updating model")
                controller.triggerUpdate()
            }.setOnSucceeded {
                println("Updating view (again)")
                updateView()
            }
        } else {
            turnLabel.text = "Your turn! Make a move..."
            setButtonsDisable(false)
        }
    }

    fun setButtonsDisable(status: Boolean) {
        heal.isDisable = status
        attack.isDisable = status
    }

    init {
        val enemyNames = controller.enemyNames
        enemyNames.forEach {
            enemyNameList.add(it)
        }
        updateView()
    }

}
package client.kotlin
import tornadofx.*

class PrimaryApp: App(ConnectView::class) {
}

fun main(args: Array<String>) {
    launch<PrimaryApp>(args)
}


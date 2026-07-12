IMPORT FXwindows

window = FXwindows.create_window("My FLUX App", 800, 600)

button = FXwindows.Button("Click Me", 100, 50)
label = FXwindows.Label("Welcome to FLUX", 20, 20)
checkbox = FXwindows.Checkbox("Enable sound", false, 20, 80)

window.add(button)
window.add(label)
window.add(checkbox)

button.text = "New Text"
PRINT button.text

window.show()

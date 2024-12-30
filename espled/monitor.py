import serial
import threading

ser = serial.Serial('/dev/ttyACM0', 115200, timeout=1)

def read_from_esp32():
    while True:
        if ser.in_waiting > 0:
            try:
                response = ser.readline().decode().strip()
                if response:
                    print(f"{response}")
            except UnicodeDecodeError:
                pass

reader_thread = threading.Thread(target=read_from_esp32, daemon=True)
reader_thread.start()

try:
    while True:
        data = input("command: ")
        if data.lower() == "exit":
            break
        ser.write((data + "\n").encode())
finally:
    ser.close()
    print("Connection closed")

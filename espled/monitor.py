
import serial
import threading

ser = serial.Serial('/dev/ttyACM0', 115200, timeout=1)

def read_from_esp32():
    buffer = b""
    while True:
        if ser.in_waiting > 0:
            # Чтение доступных данных
            data = ser.read(ser.in_waiting)
            buffer += data
            
            # Обработка сообщений с разделителем \0
            while b"\0" in buffer:
                # Разделяем по первому \0
                message, buffer = buffer.split(b"\0", 1)
                try:
                    # Декодируем и выводим сообщение
                    print(f"{message.decode().strip()}")
                except UnicodeDecodeError:
                    print(f"Received non-decodable message: {message}")

reader_thread = threading.Thread(target=read_from_esp32, daemon=True)
reader_thread.start()

try:
    while True:
        data = input("command: ")
        if data.lower() == "exit":
            break
        # Отправка JSON с разделителем \0
        ser.write((data.strip() + "\0").encode())
finally:
    ser.close()
    print("Connection closed")

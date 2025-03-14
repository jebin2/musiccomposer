import sys
import os
import json
from music_composer import MusicComposer

logger_info_code = "LogCoQ=1001"
logger_success_code = "LogCoQ=1002"
logger_error_code = "LogCoQ=1003"

try:
	# Handle optional paths argument
	input_data = json.loads(sys.argv[1])

	print(f'{logger_info_code}soundfont:{input_data.get("soundfont", "")}')
	print(f'{logger_info_code}text:{input_data.get("text", "")}')

	# Initialize SnapHound
	musicComposer = MusicComposer(soundfont_path=input_data.get("soundfont", ""))
	print(f"{logger_info_code}Music Composer Started.")

	def main(data):
		input_data["text"] = data.get("text", "")
		input_data["index"] = data.get("index", False)

		if input_data["text"]:
			file_path = musicComposer.generate_music(input_data["text"])
			print(f"{logger_success_code}{file_path}")

	def server_mode():
		main(input_data)
		while True:
			try:
				user_input = sys.stdin.readline().strip()
				if not user_input:
					continue

				args = json.loads(user_input)
				main(args)

				sys.stdout.flush()

			except (EOFError, KeyboardInterrupt):
				break

	# Start server mode
	server_mode()
except Exception as e:
	print(f"{logger_error_code}:{e}")
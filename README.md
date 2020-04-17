# Chattest-rs

Tired of all those messaging apps that use cool designs? I have the solution for you, a program that uses only ASCII characters to display the UI whilst providing the basic functionality of a messaging app.

What can you do:
- create or connect to a room
- send and recive messages
- scroll through previous messages

# TODO

The plan is to create a sort of wrapper around the `pan-curses` crate in order to be able to interact with the window more easily. Util then the program will stay in pre-release (v0.x.x)

# Warning

It currently has a problem which blocks the user interface of the user hosting the room when he sends a message.

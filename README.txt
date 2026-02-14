
# This shares some information in terms of how to develop this Secure Finance Messaging Block Application 

# Instructions: 
- Develop a rust program that will allow two servers to share message blocks from one another. 
- they must be able to securely authenticate or check each others tokens before starting to share the information. 
- When the one server sends a client request for sending a message through the other side should request for authentication token 
- one the authentication token is checked and confirmed the it will allow the message through. 
- message should be encrypted and the two servers must have a public and private keys and both servers must share there public keys to be able to encrypt and decrypt the message block.
- the program also must be a CLI application. 


# Program CLI arguements 
 * -h  this is for help comamnds 
 * -i  this is the server IP of the server to connect and send messages too 
 * -f  the text file with the message block
 * -k  the key file for the encryption
 * -s  the name of the file to be stored on the other side of the server with dateand time appended and file extension .ftt
 * -m  this is to run the interactive mode 
 * -ck this is the connect key 

# other
- make the cli with color 
- have helper notification text 
- the program must both run a server and client 
- also show the ip of the server when listaining 
- server must be listain on a particular port that is redefined with -lp [PORT-NUMBER]
- server must use a white list file with connect keys(ck keys) that it will check when accepting a connection


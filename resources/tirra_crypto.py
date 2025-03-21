from Crypto.Cipher import ChaCha20_Poly1305
from Crypto.Protocol.KDF import PBKDF2
from Crypto.Hash import SHA256
import os
import sys

# Function to generate the key using PBKDF2
def generate_key(password, salt, length=32, iterations=600):
    """Generate a key from a password using PBKDF2."""
    # PBKDF2 key derivation with HMAC-SHA256
    return PBKDF2(password, salt, dkLen=length, count=iterations, hmac_hash_module = SHA256)


def decrypt_file(encrypted_file_path, decrypted_file_path, key, nonce):
    # Open the encrypted file
    with open(encrypted_file_path, 'rb') as f:
        encrypted_data = f.read()

    # The tag (authentication tag) is the last 16 bytes of the encrypted data
    ciphertext = encrypted_data[:-16]
    tag = encrypted_data[-16:]

    # Create a ChaCha20-Poly1305 cipher object using the key and nonce
    cipher = ChaCha20_Poly1305.new(key=key, nonce=nonce)

    try:
        # Decrypt the ciphertext and authenticate the tag
        decrypted_data = cipher.decrypt_and_verify(ciphertext, tag)

        # Save the decrypted file
        with open(decrypted_file_path, 'wb') as f:
            f.write(decrypted_data)
        print("Decryption successful!")

    except ValueError:
        print("Decryption failed or tag mismatch!")


if __name__ == "__main__":
    
    # Tirra Encryption Parameters
    salt = bytes.fromhex('21bc21bc21bc2165')
    key_len = 32
    key_gen_iterations = 600
    nonce = bytes.fromhex('21bc21bc21bc21bc21bc21bc')


    # Arguments
    password = "paris"  
    encrypted_file_path = 'sync.db'
    decrypted_file_path = 'sync.db.PLAIN'
    
    # Step 1: Generate Key
      
    key = generate_key("paris", salt, key_len, key_gen_iterations)


    # Step 2: Decrypt db file
    decrypt_file(encrypted_file_path, decrypted_file_path, key, nonce)

import os
import requests
from typing import List
from typing import List, Optional
from config import Config 

class Ciaos:
    def __init__(self, config : Config):
        """
        Initialize Ciaos client with configuration parameters.
        API URL and user ID are imported from config

        self.config.api_url = "<server url>"
        config.user.id = "your_user_id"
        """

        # Get user ID from config
        self.config = config
        
        if not self.config.user_id:
            raise ValueError("User ID must not be empty")
        
        if not self.config.api_url:
            raise ValueError("API URL must not be empty")


        self.headers = {
            "User": self.config.user_id
        }

    def put(self, file_path: str, key: Optional[str] = None):
        """
        Uploads files to the server with flexible input options.

        Args:
            key (Optional[str]): Unique key for the upload. If None and file_path provided, 
                            uses filename as key.
            file_path : Path to file to upload.reads file data.

        Returns:
            requests.Response or None: The server's response or None if an error occurs.
            
        Raises:
            FileNotFoundError: If the specified file_path does not exist
            ValueError: If file_path is empty or None
        """
        if not file_path:
            raise ValueError("file_path cannot be empty or None")
            
        if not os.path.exists(file_path):
            raise FileNotFoundError(f"File not found: {file_path}")
            
        try:
            # Handle file path only case
            with open(file_path, 'rb') as file:
                file_data = file.read()
                data_list = [file_data]
                    
            # If key not provided, use filename from path
            if key is None:
                key = os.path.basename(file_path)

            # Create and send flatbuffer data
            response = requests.post(
                f"{self.config.api_url}/put/{key}",
                data=file_data,
                headers=self.headers
            )
            return response

        except requests.RequestException as e:
            print("HTTPError during upload:", e)
            return None

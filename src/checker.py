from datetime import datetime
import os
import subprocess
import sys
import re
from openai import OpenAI

from rich.console import Console

console = Console()

def extract_dafny_code(llm_response):
    """Extract the dafny code block containing the 'main' method."""
    # Find all `dafny` code blocks
    blocks = re.findall(r'```dafny\n(.*?)\n```', llm_response, re.DOTALL)
    # Loop through the blocks and find the one containing the 'method Main()'
    for block in blocks:
        if 'method main' in block:
            return block.strip()
    return None  # Return None if no 'main' method is found

def verify_with_dafny(filename):
    """Write Dafny code to a file and verify it."""
    try:
        # Run Dafny verifier as a subprocess
        result = subprocess.run(["dafny", "verify", filename], capture_output=True, text=True)
        if result.returncode == 0:
            return True, "Verification successful."
        else:
            return False, result.stdout  # Return error message in stdout
    except FileNotFoundError:
        return False, "Dafny is not installed or not in PATH."

def interact_with_llm(client, messages):
    """
    Send messages to LLM and return its response.
    client: OpenAI
    """
    completion = client.chat.completions.create(
        model="qwen-plus",
        messages=messages,
        stream=True
    )
    full_content = ""
    for chunk in completion:
        full_content += chunk.choices[0].delta.content
    return full_content

def read_file(filename):
    """Read the contents of a file."""
    if not os.path.isfile(filename):
        raise FileNotFoundError(f"File '{filename}' not found.")
    with open(filename, "r") as file:
        return file.read()

def init_llm():
    """ Intialize the LLM client and return it. """
    client = OpenAI(
        api_key=os.getenv("DASHSCOPE_API_KEY"),
        base_url="https://dashscope.aliyuncs.com/compatible-mode/v1",
    )
    messages = [
        {'role': 'system', 'content': 'You are a sophisticated Dafny programmer.'},
    ]

    return client, messages

def main(dafny_file):
    console.print("[bold]Dafny Code Checker[/bold]", style="bold blue")
    # Initialize the LLM client
    client, messages = init_llm()
    # Read the Dafny code from the file
    dafny_code = read_file(dafny_file)

    # Get the verification result
    verified, result = verify_with_dafny(dafny_file)
    
    fix_prompt = f"""Please fix the following Dafny code. Just make it syntactically right. Do not add annotations!
DON'T CHANGE ANY ASSIGNMENT AND LOGICAL IN THE CODE. DON'T CHANGE THE INITIAL VALUES OF VARIABLES.
DON'T CHANGE ANY ASSERTION, INVARIANT, MODIFIES, ENSURES, REQUIRES, ETC. e.g. when the initial value of a variable is *, just don't change it.
and when there is `(* - 3)` which is not allowed in Dafny, fix it!
 JUST FIX THE SYNTAX ERRORS.
And please wrap the code in a
```dafny
    code goes here
``` code block. The code you need to fix is:\n{dafny_code}
    The error message is:\n{result}"""
    messages.append({'role': 'user', 'content': fix_prompt})

    # Interact with LLM
    llm_response = interact_with_llm(client, messages)
    # Extract the Dafny code from the response
    new_dafny_code = extract_dafny_code(llm_response)
    if new_dafny_code is None:
        raise ValueError("Could not extract Dafny code from LLM response.")
    
    # Write the new Dafny code to a file
    with open(dafny_file, "w") as file:
        file.write(new_dafny_code)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python checker.py <dafny-file>")
        sys.exit(1)
    dafny_file = sys.argv[1]
    main(dafny_file)
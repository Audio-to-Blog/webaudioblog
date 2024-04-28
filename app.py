from flask import Flask, render_template, request, jsonify
import boto3
import requests
import os
import uuid
from werkzeug.utils import secure_filename


app = Flask(__name__)
processing_status = {}  # Simple in-memory storage

# AWS S3 Configuration
S3_BUCKET = 'transcribe-ids721'
s3 = boto3.client('s3', aws_access_key_id=os.getenv('S3_KEY'), aws_secret_access_key=os.getenv('S3_SECRET'))

@app.route('/')
def index():
    return render_template('index.html')

@app.route('/upload', methods=['POST'])
def upload():
    file = request.files['file']
    if file:
        filename = secure_filename(file.filename)
        s3.upload_fileobj(file, S3_BUCKET, filename, ExtraArgs={"ContentType": file.content_type})
        return jsonify({"message": "File uploaded successfully", "filename": filename})
    return jsonify({"error": "Failed to upload file"}), 400

@app.route('/process', methods=['POST'])
def process_file():
    filename = request.json.get('filename')
    if filename:
        process_id = str(uuid.uuid4())
        processing_status[process_id] = {"complete": False, "result": None}
        data = {
            "input": "{\"filename\": \"" + "s3://transcribe-ids721/" + filename + "\"}",
            "name": "Execution-" + process_id,
            "stateMachineArn": "arn:aws:states:us-east-1:718203338152:stateMachine:transcribe"
        }
        headers = {'Content-Type': 'application/json'}
        url = 'https://wrnqr49qhe.execute-api.us-east-1.amazonaws.com/beta/execution'
        requests.post(url, json=data, headers=headers)
        return jsonify({"message": "Processing started", "processId": process_id})
    return jsonify({"error": "Filename is missing"}), 400

@app.route('/callback', methods=['POST'])
def callback():
    data = request.json
    process_id = data.get('name')
    if process_id in processing_status:
        processing_status[process_id] = {"complete": True, "result": data}
    return jsonify({"status": "success", "data": data}), 200

@app.route('/status', methods=['GET'])
def check_status():
    process_id = request.args.get('processId')
    if process_id in processing_status and processing_status[process_id]['complete']:
        return jsonify({"complete": True, "result": processing_status[process_id]['result']})
    else:
        return jsonify({"complete": False})

if __name__ == '__main__':
    app.run(debug=True)

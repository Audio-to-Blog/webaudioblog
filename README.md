# Operationalizing Audio Transcription and Blog Generation - Rust Web Service

This is a simple Rust Actix web application that allows users to upload the audio file of a conversation/interview and get a blog summarizing what was discussed. The user uploaded audio file is automatically stored to an AWS S3 bucket. The audio file is processed using AWS Step Functions, which uses a speech to text model and an LLM to generate automatically generate blog about the conversation. 

## Deployment (Local)

1. Clone the repository.
2. Install the required dependencies using `pip install -r requirements.txt`.
3. Set up your AWS credentials in your environment variables:
    - `S3_KEY`: Your AWS S3 access key.
    - `S3_SECRET`: Your AWS S3 secret key.
4. Run `uvicorn main:app --reload` in your terminal.

You will now be able to access the service at `http://127.0.0.1:8080/`.

## Deployment (Vercel)

To deploy the application on Vercel, create a new Project and import this git repository. Vercel will then parse the `vercel.json` document included in the repo and provide you with a link to access the service.

## Usage (Local)

1. Navigate to the service url (see above) in your web browser.
2. Click on the "Choose File" button to select a file for upload.
3. Click on the "Upload" button to upload the selected file to the AWS S3 bucket.
4. Once the file is uploaded, you will receive a message confirming the successful upload and the filename.
5. To process the uploaded file, navigate to `https://audio-to-blog.vercel.app/process` and enter the filename in the provided field.
6. Click on the "Process" button to start the processing. The processing status will be updated in real-time.

## Note

This application is for demonstration purposes only. It is not intended for production use.

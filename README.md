# Operationalizing Audio Transcription and Blog Generation - Rust Web Service

This is a simple Rust Actix web application that allows users to upload the audio file of a conversation/interview and get a blog summarizing what was discussed. 

## Workflow

The user uploaded audio file is automatically stored to an AWS S3 bucket. The audio file is processed using AWS Step Functions, which uses a speech to text model and an LLM to generate automatically generate blog about the conversation. 

### Inference ML Model
Amazon Transcribe is used to transcribe audio to text. The model is efficient and capable of inferences based on input data it receives. You can find details of Amazon Transcribe [here](https://aws.amazon.com/pm/transcribe/?gclid=CjwKCAjwxLKxBhA7EiwAXO0R0K6QsdXV2XsDvlKZim3tfYUJRmjjIXDTcCbMHlZT-MEk5SGwjxCDpxoC6OoQAvD_BwE&trk=aae0a267-33fa-4d21-a4d5-30b7b3fd731e&sc_channel=ps&ef_id=CjwKCAjwxLKxBhA7EiwAXO0R0K6QsdXV2XsDvlKZim3tfYUJRmjjIXDTcCbMHlZT-MEk5SGwjxCDpxoC6OoQAvD_BwE:G:s&s_kwcid=AL!4422!3!648922763916!e!!g!!amazon%20transcription!19597968945!143908652045)

#### Rust Web Service for Model Inferences
A web service in Rust is developed such that it automatically stores the user's audio file in an AWS S3 bucket. The /process API request automatically calls an AWS Step Function, which takes the file from the S3 bucket, transcribes it using AWS Transcribe, creates a blog using GPT-4 and calls a callback POST request to communicate the final blog back to the web service.

### Containerization and Kubernetes 
The Rust web service using Docker preparing it for deployment. The containerized service is deploted a Kubernetes cluster, specifically [AWS ECS](https://aws.amazon.com/ecs/).

### CI/CD Pipeline
Github Actions continuous integration and continuous deployment (CI/CD) pipeline is used automate the testing, building, and deployment of the web service.

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

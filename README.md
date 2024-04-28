# Operationalizing Audio Transcription and Blog Generation - Rust Web Service

This is a simple Rust Actix web application that allows users to upload the audio file of a conversation/interview and get a blog summarizing what was discussed. 

Find the website here: https://blogger.sanjeev.one/

## Demo

## Components

The user uploaded audio file is automatically stored to an AWS S3 bucket. The audio file is processed using AWS Step Functions, which uses an open source speech to text model and LLM to generate automatically generate blog about the conversation. 

#### Inference ML Model
Amazon Transcribe is used to transcribe the user's conversation/interview to text. The model is efficient and capable of text inferences based on input audio it receives. You can find details of Amazon Transcribe [here](https://aws.amazon.com/pm/transcribe/?gclid=CjwKCAjwxLKxBhA7EiwAXO0R0K6QsdXV2XsDvlKZim3tfYUJRmjjIXDTcCbMHlZT-MEk5SGwjxCDpxoC6OoQAvD_BwE&trk=aae0a267-33fa-4d21-a4d5-30b7b3fd731e&sc_channel=ps&ef_id=CjwKCAjwxLKxBhA7EiwAXO0R0K6QsdXV2XsDvlKZim3tfYUJRmjjIXDTcCbMHlZT-MEk5SGwjxCDpxoC6OoQAvD_BwE:G:s&s_kwcid=AL!4422!3!648922763916!e!!g!!amazon%20transcription!19597968945!143908652045)

#### Rust Web Service for Model Inferences
The web service, developed in Rust, automatically stores the user's audio file in an AWS S3 bucket. The /process API request automatically calls an AWS Step Function, which takes the file from the S3 bucket, transcribes it using AWS Transcribe, creates a blog using LLAMA 70B LLM and calls a callback POST request to communicate the final blog back to the web service. The web service continously polls the callback until the blog is ready. See a brief documentation of the [AWS Step Function](https://github.com/Audio-to-Blog/AWS-Step-Function)

See the Rust code in the 'main' folder.

#### Containerization, Kubernetes, and CI/CD Pipeline

The Rust web service is containerized using Docker, allowing for seamless deployment across environments. Deployment is managed on an AWS EKS cluster configured with two subnets in different availability zones for improved reliability. The compute resources are allocated using AWS Fargate, which abstracts server management, using the following specifications:
- **CPU Requests**: 256m (a quarter core)
- **Memory Requests**: 512Mi (512 MiB)
- **CPU Limits**: 512m (half a core)
- **Memory Limits**: 1Gi (1 GiB)

For deployment strategy, we use a RollingUpdate:
- **Max Surge**: 1 (one additional pod during the update)
- **Max Unavailable**: 1 (one pod can be unavailable during the update)

Our CI/CD pipeline, implemented via GitHub Actions, builds the Docker image and pushes it to AWS ECR. The image is then deployed to the AWS EKS cluster. The Fargate configuration uses a private subnet, enhancing the security of the deployment environment.

#### Monitoring and Metrics
 Amazon CloudWatch is enabled on AWS Lambda functions for log group creation, log streams, and log events. For debugging with rust web service, the code is augmented with dense tracing and console outputs. The metrics are kept track of through the respective services' AWS portals. 

## Deployment (Vercel)

1. Clone the repository. Make sure Rust is installed.
2. 
3. Set up your AWS credentials in your environment variables:
    - `S3_KEY`: Your AWS S3 access key.
    - `S3_SECRET`: Your AWS S3 secret key.
4. Run `uvicorn main:app --reload` in your terminal.

You will now be able to access the service at `http://127.0.0.1:8080/`.

To deploy the application on Vercel, create a new Project and import this git repository. Vercel will then parse the `vercel.json` document included in the repo and provide you with a link to access the service.

## Usage (Local)

1. Navigate to the service url (see above) in your web browser.
2. Click on the "Choose File" button to select a file for upload.
3. Click on the "Upload" button to upload the selected file to the AWS S3 bucket.
4. Once the file is uploaded, you will receive a message confirming the successful upload and the filename.
5. The file is automatically processed by the rust web service with `https://blogger.sanjeev.one/process` and parsing filename.
6. Wait for the audio to be transcribed and turned into a blog. The screen will display "Processing" with a spinning wheel.
7. Blog will display.

## Team Size and Makeup
The creators are:
Hadi Chaudhri (Rust Web Service), Peter Liu (Rust Web Service), Benjamin Chauhan (Lambda Functions), Sanjeev (Step Function, CICD, k8s and Dockerization)



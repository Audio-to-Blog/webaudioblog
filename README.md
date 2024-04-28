# Operationalizing Audio Transcription and Blog Generation - Rust Web Service

This is a simple Rust Actix web application that allows users to upload the audio file of a conversation/interview and get a blog summarizing what was discussed. 

Find the website here: https://blogger.sanjeev.one/

## Demo

https://youtu.be/tHNcZzEcKlg


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


## Deployment Setup (AWS EKS using GitHub Actions)

1. **Clone the Repository**:
   - Ensure you have Rust installed on your local machine.
   - Clone the GitHub repository to your local environment using `git clone [repository-url]`.

2. **Configure AWS Credentials**:
   - Set up your AWS credentials securely. Use environment variables on your local machine or configure them as secrets in GitHub for the repository:
     - `AWS_ACCESS_KEY_ID`: Your AWS access key ID.
     - `AWS_SECRET_ACCESS_KEY`: Your AWS secret key.
     - `AWS_DEFAULT_REGION`: Your AWS region (e.g., `us-east-1`).

3. **Local Testing (Optional)**:
   - Build and run your Docker container locally to ensure everything is working as expected before deploying:
     - `docker build -t your-image-name .`
     - `docker run -p 8080:8080 your-image-name`
Adding secrets to AWS EKS involves securely managing sensitive data like API keys, credentials, and configuration details. You can use Kubernetes secrets or integrate with AWS Secrets Manager for enhanced security. Here are detailed steps to add secrets to your AWS EKS environment using both Kubernetes secrets and AWS Secrets Manager:

### Setup Kubernetes Secrets

4. **Create a Kubernetes Secret**:
   - Create a YAML file to define your secret. For example, `my-secret.yaml`:
     ```yaml
     apiVersion: v1
     kind: Secret
     metadata:
       name: my-secret
     type: Opaque
     data:
       AWS_ACCESS_KEY_ID: [base64 encoded value]
       AWS_SECRET_ACCESS_KEY: [base64 encoded value]
     ```
   - Replace `[base64 encoded value]` with your base64-encoded AWS credentials. You can encode your credentials using `echo -n 'your-value' | base64`.

5. **Apply the Secret to Your Cluster**:
   - Use `kubectl` to apply the secret to your cluster:
     ```bash
     kubectl apply -f my-secret.yaml
     ```

6. **Reference the Secret in Your Deployment**:
   - Modify your deployment YAML to use the secret as environment variables:
     ```yaml
     apiVersion: apps/v1
     kind: Deployment
     metadata:
       name: my-deployment
     spec:
       containers:
       - name: my-container
         image: my-image
         env:
           - name: AWS_ACCESS_KEY_ID
             valueFrom:
               secretKeyRef:
                 name: my-secret
                 key: AWS_ACCESS_KEY_ID
           - name: AWS_SECRET_ACCESS_KEY
             valueFrom:
               secretKeyRef:
                 name: my-secret
                 key: AWS_SECRET_ACCESS_KEY
     ```

7. **Push Changes**:
   - Make any necessary changes to your application or Dockerfile.
   - Push your changes to the main branch of your GitHub repository:
     - `git add .`
     - `git commit -m "Prepare for deployment"`
     - `git push origin main`

8. **Automatic Deployment**:
   - The push to the main branch will trigger the GitHub Actions workflow.
   - Monitor the workflow execution within the GitHub Actions tab to ensure the build and deployment processes complete successfully.

9. **Verify Deployment**:
   - Once the deployment is successful, check the application's functionality by accessing the provided Load Balancer URL from AWS EKS.


## Usage (Local)

1. Navigate to the service url in your web browser. If you do not have your own, use https://blogger.sanjeev.one/ .
2. Click on the "Choose File" button to select a file for upload.
3. Click on the "Upload" button to upload the selected file to the AWS S3 bucket.
4. Once the file is uploaded, you will receive a message confirming the successful upload and the filename.
5. The file is automatically processed by the rust web service with `https://blogger.sanjeev.one/process` and parsing filename.
6. Wait for the audio to be transcribed and turned into a blog. The screen will display "Processing" with a spinning wheel.
7. Blog will display.

## Team Size and Makeup
The creators are:
Hadi Chaudhri (Rust Web Service), Peter Liu (Rust Web Service), Benjamin Chauhan (Lambda Functions), Sanjeev (Step Function, CICD, k8s and Dockerization)





pipeline {
  agent any

  environment {
    DOCKERHUB = credentials('DockerHub')
  }

  stages{

    stage('Docker Login') {
      steps {
        sh '''
          echo $DOCKERHUB_PSW | docker login -u $DOCKERHUB_USR --password-stdin
        '''
      }
    }

  stage('Pull images') {
      steps {
        sh 'docker compose pull'
      }
    }
      

    stage('Deploy') {
      steps {
       // runs the same stack of dependensies as your local machine
        sh 'docker compose up -d'
        sh 'docker compose down'
      }
    }
  }
}
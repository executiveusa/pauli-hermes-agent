FROM node:20-alpine
WORKDIR /app
COPY package.json ./
RUN npm install
COPY src ./src
EXPOSE 3000
CMD ["npm","run","dev","--","-H","0.0.0.0","-p","3000"]

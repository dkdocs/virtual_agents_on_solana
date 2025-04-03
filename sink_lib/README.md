## Substreams Sink Examples - JS (Node)

This example consumes a Substreams package (specifically, the [Ethereum Explorer](https://substreams.dev/streamingfast/ethereum-explorer/v0.1.2)) using the Substreams JS library in NodeJS.

The [API token](https://substreams.streamingfast.io/documentation/consume/authentication) is provided through an environment variable:

```javascript
const TOKEN = process.env.SUBSTREAMS_API_TOKEN
```

If you don't have it, set the `SUBSTREAMS_API_TOKEN` environment variable. Then, you can easily get started:

1. Install the dependencies.

    ```bash
    npm install
    ```

1. Run the script.

    ```bash
    node index.js
    ```

# Data Population Guide by harshrjjpt

Follow these steps to populate data efficiently:  

## **Prerequisites**  

- Ensure you have Node.js and substream sink js installed.
- Install **Substreams.js** from the official repository: [Substreams.js on GitHub](https://github.com/substreams-js/substreams-js).  
- Obtain your Substream JWT token using the [Substreams Authentication Guide](https://docs.substreams.dev/reference-material/substreams-cli/authentication#step-1-obtain-a-jwt-token).  

---

## **Step 1: Create Database and Package File**  

1. Navigate to the **`substream`** directory.  
2. Follow the instructions in the `Makefile` to create the database and generate the package file.  

---

## **Step 2: Install Dependencies**  

1. Move to the **`sink_lib`** directory:  
    ```bash
    cd sink_lib
    ```  
2. Install all required dependencies using:  
    ```bash
    npm install
    ```  

---

## **Step 3: Set Environment Variables**  

- Configure your environment variables by running:  
    ```bash
    source set_environment_vars.sh
    ```  

---

## **Step 4: Authenticate with Substreams**  

- Export your Substreams API token:  
    ```bash
    export SUBSTREAMS_API_TOKEN="{YOUR_JWT_TOKEN}"
    ```  
- For guidance on obtaining your JWT token, refer to the [Substreams Documentation](https://docs.substreams.dev/reference-material/substreams-cli/authentication#step-1-obtain-a-jwt-token).  

---

## **Step 5: Start Data Population**  

- Run the following script to initiate data population:  
    ```bash
    npm run sinkData
    ```  

---

✅ **You’re all set!**

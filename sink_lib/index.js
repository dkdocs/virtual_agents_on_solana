import {
	createRequest,
	streamBlocks,
	createAuthInterceptor,
	createRegistry,
	applyParams,
	createSubstream,
} from "@substreams/core";
import {createConnectTransport} from "@connectrpc/connect-node";
import {getCursor} from "./cursor.js";
import {isErrorRetryable} from "./error.js";
import fs from "fs/promises";
import {handleResponseMessage} from "./handlers.js";
import {Labels} from "./constants.js";
import {getCommaSeparatedTokens, getCurrentBlockNumber, updateCurrentBlockNumber} from "./clickhouseWrapper.js";
import { logger } from './logger.js';

const args = process.argv.slice(2);
const params = {};

args.forEach((arg) => {
	const [key, value] = arg.split("=");
	if (key.startsWith("--")) {
		params[key.slice(2)] = value;
	}
});

const TOKEN = process.env.SUBSTREAMS_API_TOKEN;
const ENDPOINT = process.env.NETWORK;
const SPKG = process.env.PACKAGE_FILE_PATH;

// Get initial block number from database or use default
const getInitialBlockNumber = async () => {
	const dbBlockNumber = await getCurrentBlockNumber();
	if (dbBlockNumber) {
		logger.info(`Using block number from database: ${dbBlockNumber}`);
		return dbBlockNumber;
	}
	return params?.startBlock || Labels.initialBlockNumber;
};

let START_BLOCK = await getInitialBlockNumber() || params?.startBlock || Labels.initialBlockNumber;

let STOP_BLOCK = params?.stopBlock || Labels.blocksGap;

const main = async (moduleName) => {
	const MODULE = moduleName === Labels.tokensSubstreamPackage ? "map_filter_transactions" : "map_solana_token_events";
	const pkg = await fetchPackage();

	if (moduleName === Labels.transferSubstreamPackage) {
		const tokens = await getCommaSeparatedTokens();
		if (tokens.length > 0) {
			applyParams([`map_solana_token_events=${tokens}`], pkg.modules?.modules);
		}
	}

	const registry = createRegistry(pkg);
	const transport = createConnectTransport({
		baseUrl: ENDPOINT,
		interceptors: [createAuthInterceptor(TOKEN)],
		useBinaryFormat: true,
		jsonOptions: {typeRegistry: registry},
	});

	while (true) {
		try {
			logger.info(`Processing from block ${START_BLOCK} to ${STOP_BLOCK} for ${MODULE} module`);
			await stream(pkg, registry, transport, MODULE, START_BLOCK, STOP_BLOCK, moduleName);
			break;
		} catch (e) {
			if (!isErrorRetryable(e)) {
				logger.error(`Fatal error occurred in ${MODULE}: ${e}`);
				throw e;
			}
			logger.error(`Retryable error occurred in ${MODULE}: ${e}`);
			logger.info(`Retrying in 5 seconds...`);
			await new Promise((resolve) => setTimeout(resolve, 5000));
		}
	}
};

const fetchPackage = async () => {
	const fileBuffer = await fs.readFile(SPKG);
	return createSubstream(fileBuffer);
};

const stream = async (pkg, registry, transport, moduleName, startBlock, stopBlock, activePackage) => {
	const request = createRequest({
		substreamPackage: pkg,
		outputModule: moduleName,
		productionMode: true,
		startBlockNum: startBlock,
		stopBlockNum: stopBlock,
		startCursor: (await getCursor(activePackage)) ?? undefined,
	});

	for await (const response of streamBlocks(transport, request)) {
		await handleResponseMessage(response.message, registry, activePackage);
	}
};

const runSequentially = async () => {
	while (true) {
		logger.info(`START_BLOCK: ${START_BLOCK}, STOP_BLOCK: ${STOP_BLOCK}`);

		await processModules();

		START_BLOCK = parseInt(START_BLOCK) + parseInt(Labels.blocksGap);

		logger.info(`Updated START_BLOCK: ${START_BLOCK}, STOP_BLOCK: ${STOP_BLOCK}`);
	}
};

const processModules = async () => {
	await main("tokenPackage");
	await main("transferPackage");
};

runSequentially();

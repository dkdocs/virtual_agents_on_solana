const LOG_LEVELS = {
    DEBUG: 0,
    INFO: 1,
    WARN: 2,
    ERROR: 3
};

const getCurrentLogLevel = () => {
    const level = process.env.LOG_LEVEL?.toUpperCase() || 'INFO';
    return LOG_LEVELS[level] || LOG_LEVELS.INFO;
};

const getTimestamp = () => {
    return new Date().toISOString();
};

const formatMessage = (level, message) => {
    return `[${getTimestamp()}] [${level}] ${message}`;
};

const shouldLog = (level) => {
    return LOG_LEVELS[level] >= getCurrentLogLevel();
};

export const logger = {
    info: (message) => {
        if (shouldLog('INFO')) {
            console.log(formatMessage('INFO', message));
        }
    },
    error: (message) => {
        if (shouldLog('ERROR')) {
            console.error(formatMessage('ERROR', message));
        }
    },
    warn: (message) => {
        if (shouldLog('WARN')) {
            console.warn(formatMessage('WARN', message));
        }
    },
    debug: (message) => {
        if (shouldLog('DEBUG')) {
            console.debug(formatMessage('DEBUG', message));
        }
    }
}; 
import { APP_CONFIG } from '../data/config';

/**
 * Get application metadata
 * @returns {Object} App metadata including name, version, author, etc.
 */
export const getAppInfo = () => {
    return {
        name: APP_CONFIG.app.name,
        displayName: APP_CONFIG.app.displayName,
        version: APP_CONFIG.app.version,
        author: APP_CONFIG.app.author,
        repository: APP_CONFIG.app.repository,
        fullTitle: `${APP_CONFIG.app.displayName} v${APP_CONFIG.app.version}`
    };
};

/**
 * Get app version
 * @returns {string} Current app version
 */
export const getAppVersion = () => APP_CONFIG.app.version;

/**
 * Get app name
 * @returns {string} App display name
 */
export const getAppName = () => APP_CONFIG.app.displayName;

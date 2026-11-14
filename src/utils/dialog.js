import { STRINGS, formatString } from '../data/strings';

/**
 * Show a confirmation dialog
 * @param {string} message - Message to display
 * @param {Object} values - Values to interpolate into message
 * @returns {boolean} - User's response
 */
export const confirmDialog = (message, values = {}) => {
    const formattedMessage = formatString(message, values);
    return window.confirm(formattedMessage);
};

/**
 * Show an alert dialog
 * @param {string} message - Message to display
 * @param {Object} values - Values to interpolate into message
 */
export const alertDialog = (message, values = {}) => {
    const formattedMessage = formatString(message, values);
    window.alert(formattedMessage);
};

/**
 * Show a prompt dialog
 * @param {string} message - Message to display
 * @param {string} defaultValue - Default input value
 * @returns {string|null} - User's input or null if cancelled
 */
export const promptDialog = (message, defaultValue = "") => {
    return window.prompt(message, defaultValue);
};

/**
 * Confirm deletion
 * @param {string} itemName - Name of item to delete
 * @returns {boolean}
 */
export const confirmDelete = (itemName) => {
    return confirmDialog(STRINGS.files.confirmDelete, { item: itemName });
};

/**
 * Confirm app uninstall
 * @param {string} appName - Name of app to uninstall
 * @returns {boolean}
 */
export const confirmUninstall = (appName) => {
    return confirmDialog(STRINGS.apps.confirmUninstall, { app: appName });
};

/**
 * Confirm clear app data
 * @param {string} appName - Name of app
 * @returns {boolean}
 */
export const confirmClearData = (appName) => {
    return confirmDialog(STRINGS.apps.confirmClearData, { app: appName });
};

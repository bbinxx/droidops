import { STRINGS, formatString } from '../data/strings';

/**
 * Custom hook to access app strings
 * @returns {Object} - Strings object and format function
 */
export const useStrings = () => {
    return {
        strings: STRINGS,
        format: formatString
    };
};

// Convenience exports for common string sections
export const useNavStrings = () => STRINGS.navigation;
export const useDeviceStrings = () => STRINGS.device;
export const useErrorStrings = () => STRINGS.errors;
export const useCommonStrings = () => STRINGS.common;

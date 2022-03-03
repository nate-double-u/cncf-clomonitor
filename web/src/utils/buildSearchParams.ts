import isNull from 'lodash/isNull';

import { SearchFiltersURL } from '../types';

interface F {
  [key: string]: string[];
}

const WHITELISTED_FILTER_KEYS = [
  'category', // Project category
  'maturity', // Project maturity
  'rating', // Quality rating
];

/**
 * Translates search query string string to search filters
 *
 * @param p - query string of the search page
 * @returns The search filters
 *
 * @example
 * ```
 * // Returns { text: undefined, filters: { maturity: ['1'] }, pageNumber: 1 }:
 * buildSearchParams('?page=1&maturity=1');
 * ```
 */
const buildSearchParams = (p: URLSearchParams): SearchFiltersURL => {
  let filters: F = {};

  p.forEach((value, key) => {
    if (WHITELISTED_FILTER_KEYS.includes(key)) {
      const values = filters[key] || [];
      values.push(value);
      filters[key] = values;
    }
  });

  return {
    text: p.has('text') ? p.get('text')! : undefined,
    filters: { ...filters },
    pageNumber: p.has('page') && !isNull(p.get('page')) ? parseInt(p.get('page')!) : 1,
  };
};

export default buildSearchParams;

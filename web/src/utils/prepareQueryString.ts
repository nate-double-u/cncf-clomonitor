import { isEmpty, isUndefined } from 'lodash';

import { SearchFiltersURL } from '../types';

/**
 * Prepares search query string
 *
 * @param query - {@link SearchFiltersURL}
 * @returns The query string
 *
 * @example
 * ```
 * // Returns '?category=0&category=2&text=test&page=2':
 * prepareQueryString({ pageNumber: 2, text: 'test', filters: { category: ['0', '2'] } });
 * ```
 */
const prepareQueryString = (query: SearchFiltersURL): string => {
  const q = new URLSearchParams();
  if (!isUndefined(query.filters) && !isEmpty(query.filters)) {
    Object.keys(query.filters).forEach((filterId: string) => {
      return query.filters![filterId].forEach((id: string | number) => {
        q.append(filterId, id.toString());
      });
    });
  }
  if (!isUndefined(query.text) && query.text !== '') {
    q.set('text', query.text);
  }
  q.set('page', query.pageNumber.toString());
  return `?${q.toString()}`;
};

export default prepareQueryString;

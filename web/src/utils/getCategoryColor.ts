/**
 * Get rating color
 *
 * @param value - score value
 * @returns The correct color: green | yellow | orange | red
 *
 * @example
 * ```
 * // Returns 'orange':
 * getCategoryColor(60);
 * ```
 */
const getCategoryColor = (value: number): string => {
  if (value < 25) {
    return 'red';
  } else if (value >= 25 && value < 50) {
    return 'orange';
  } else if (value >= 50 && value < 75) {
    return 'yellow';
  }
  return 'green';
};

export default getCategoryColor;

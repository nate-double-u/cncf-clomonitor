import { useEffect } from 'react';

/**
 * Hook - block body scroll.
 *
 * @remarks
 * If you want to block scroll only for one breakpoint,
 * consider using {@link breakPoint}.
 *
 * @param blocked - Block or not scroll
 * @param elType - slider | modal
 * @param breakPoint - Optional parameter for breakpoint
 */
export default function useBodyScroll(blocked: boolean, elType: string, breakPoint?: string) {
  const className = `noScroll-${elType}`;
  const bkClassName = breakPoint ? `noScroll-${breakPoint}` : '';

  useEffect(() => {
    if (blocked) {
      document.body.classList.add(className);
      if (breakPoint) {
        document.body.classList.add(bkClassName);
      }
    } else {
      document.body.classList.remove(className);
      if (breakPoint) {
        document.body.classList.remove(bkClassName);
      }
    }

    return () => {
      document.body.classList.remove(className);
      if (breakPoint) {
        document.body.classList.remove(bkClassName);
      }
    };
  }, [bkClassName, blocked, breakPoint, className]);
}


const defaultOptions= {
    root: null,
    rootMargin: '0px',
    threshold: 0,
    unobserveOnEnter: false,
  };
  
  const createEvent = (
    name,
    detail
  )=> new CustomEvent(name, { detail });
  
  export function inView(node, options= {}) {
    const { root, rootMargin, threshold, unobserveOnEnter } = {
      ...defaultOptions,
      ...options,
    };
  
    let prevPos = {
      x: undefined,
      y: undefined,
    };
  
    let scrollDirection = {
      vertical: undefined,
      horizontal: undefined,
    };
  
    if (typeof IntersectionObserver !== 'undefined' && node) {
      const observer = new IntersectionObserver(
        (entries, _observer) => {
          entries.forEach((singleEntry) => {
            if (prevPos.y > singleEntry.boundingClientRect.y) {
              scrollDirection.vertical = 'up';
            } else {
              scrollDirection.vertical = 'down';
            }
  
            if (prevPos.x > singleEntry.boundingClientRect.x) {
              scrollDirection.horizontal = 'left';
            } else {
              scrollDirection.horizontal = 'right';
            }
  
            prevPos = {
              y: singleEntry.boundingClientRect.y,
              x: singleEntry.boundingClientRect.x,
            };
  
            const detail = {
              inView: singleEntry.isIntersecting,
              entry: singleEntry,
              scrollDirection,
              node,
              observer: _observer,
            };
  
            node.dispatchEvent(createEvent('change', detail));
  
            if (singleEntry.isIntersecting) {
              node.dispatchEvent(createEvent('enter', detail));
  
              unobserveOnEnter && _observer.unobserve(node);
            } else {
              node.dispatchEvent(createEvent('leave', detail));
            }
          });
        },
        {
          root,
          rootMargin,
          threshold,
        }
      );
  
      observer.observe(node);
  
      return {
        destroy() {
          observer.unobserve(node);
        },
      };
    }
  }
  
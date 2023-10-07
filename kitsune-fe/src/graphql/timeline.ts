import { useQuery } from '@urql/vue';

import { graphql } from './types';

function getHome(): unknown {
  return useQuery({
    query: graphql(`
      query getHomeTimeline {
        homeTimeline(before: "00000000-0000-0000-0000-000000000000")
          @_relayPagination(mergeMode: "after") {
          nodes {
            id
            subject
            content
            url
            account {
              id
              displayName
              username
              url
            }
          }
          pageInfo {
            startCursor
            endCursor
          }
        }
      }
    `),
  });
}

export { getHome };

import { useQuery } from '@urql/vue';

import { ComputedRef, computed } from 'vue';

import { graphql } from '../graphql/types';
import { Instance } from './types/graphql';

function useInstanceInfo(): ComputedRef<Instance | undefined> {
  const { data } = useQuery({
    query: graphql(`
      query getInstanceInfo {
        instance {
          description
          domain
          localPostCount
          registrationsOpen
          name
          userCount
          version
        }
      }
    `),
  });

  return computed(() => data.value?.instance);
}

export { useInstanceInfo };

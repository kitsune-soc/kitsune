import { useQuery } from '@urql/vue';

import { ComputedRef, Ref, computed } from 'vue';

import { graphql } from '../graphql/types';

type InstanceInfo = {
  description: string;
  domain: string;
  localPostCount: number;
  name: string;
  registrationsOpen: boolean;
  userCount: number;
  version: string;
};

function useInstanceInfo(): ComputedRef<InstanceInfo | undefined> {
  const { data }: { data: Ref<{ instance: InstanceInfo } | undefined> } =
    useQuery({
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

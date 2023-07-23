import { useQuery } from '@vue/apollo-composable';
import gql from 'graphql-tag';
import { Ref, toRefs } from 'vue';

type InstanceInfo =
  | {
      instance: {
        description: string;
        domain: string;
        localPostCount: number;
        name: string;
        registrationsOpen: boolean;
        userCount: number;
        version: string;
      };
    }
  | undefined;

function useInstanceInfo(): Ref<InstanceInfo> {
  let { result } = useQuery(gql`
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
  `);

  return result;
}

export { useInstanceInfo };

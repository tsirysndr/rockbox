import { gql } from "@apollo/client";

export const GET_ROCKBOX_VERSION = gql`
  query GetRockboxVersion {
    rockboxVersion
  }
`;

export const GET_GLOBAL_STATUS = gql`
  query GetGlobalStatus {
    globalStatus {
      resumeIndex
      resumeCrc32
      resumeOffset
      resumeElapsed
    }
  }
`;

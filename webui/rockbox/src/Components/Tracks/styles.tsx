import styled from "@emotion/styled";

export const Container = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  height: 100%;
`;

export const MainView = styled.div`
  display: flex;
  flex: 1;
  flex-direction: column;
  padding: 20px;
`;

export const Title = styled.div`
  font-size: 24px;
  font-family: RockfordSansMedium;
  margin-bottom: 20px;
`;

export const IconButton = styled.button`
  background-color: transparent;
  cursor: pointer;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  &:hover {
    opacity: 0.6;
  }
`;

export const Hover = styled.button`
  color: transparent;
  background-color: transparent;
  border: none;
  opacity: 1 !important;
  cursor: pointer;
  &:hover,
  &:focus {
    color: #000;
    opacity: 1 !important;
  }
`;

export const ButtonGroup = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

export const ContentWrapper = styled.div`
  padding-left: 30px;
  padding-right: 30px;
  overflow-y: auto;
  height: calc(100vh - 60px);
`;

export const AlbumCover = styled.img`
  height: 48px;
  width: 48px;
`;

import { Chat } from "@/components/rooms/chat";
import { Editors } from "@/components/rooms/editors";
import { RoomProvider } from "@/components/rooms/room-provider";
import { SidePanel } from "@/components/rooms/side-panel";
import { GridItem, SimpleGrid } from "@chakra-ui/react";
import { NextPage } from "next";

const RoomPage: NextPage = () => {
  return (
    <RoomProvider>
      <SimpleGrid gridTemplateColumns={["4fr 6fr 3fr"]} h="100vh">
        <GridItem bg="blackAlpha.300">
          <SidePanel />
        </GridItem>

        <GridItem bg="blackAlpha.200">
          <Editors />
        </GridItem>

        <GridItem bg="blackAlpha.300">
          <Chat />
        </GridItem>
      </SimpleGrid>
    </RoomProvider>
  );
};

export default RoomPage;

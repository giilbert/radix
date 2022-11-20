import { RoomProvider } from "@/components/rooms/room-provider";
import { SidePanel } from "@/components/rooms/side-panel";
import { Box, GridItem, SimpleGrid } from "@chakra-ui/react";
import { NextPage } from "next";
import { useRouter } from "next/router";

const RoomPage: NextPage = () => {
  const router = useRouter();

  return (
    <RoomProvider>
      <SimpleGrid gridTemplateColumns={["4fr 6fr 3fr"]} h="100vh">
        <GridItem bg="blackAlpha.300">
          <SidePanel />
        </GridItem>

        <GridItem bg="blackAlpha.200"></GridItem>

        <GridItem bg="blackAlpha.300"></GridItem>
      </SimpleGrid>
    </RoomProvider>
  );
};

export default RoomPage;

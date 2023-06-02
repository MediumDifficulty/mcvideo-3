package yes.mediumdifficulty.extractor.client;

import net.fabricmc.api.ClientModInitializer;
import net.minecraft.block.MapColor;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.nio.file.Files;
import java.nio.file.OpenOption;
import java.nio.file.Paths;
import java.nio.file.StandardOpenOption;
import java.util.ArrayList;
import java.util.List;

public class ExtractorClient implements ClientModInitializer {
    private static final String bakedFilePath = "./extracted/closest_colours.dat";

    @Override
    public void onInitializeClient() {
        System.out.println("Starting extraction...");

        Field[] declaredFields = MapColor.class.getDeclaredFields();
        List<Integer> colours = new ArrayList<Integer>();

        for (Field field : declaredFields) {
            if (Modifier.isStatic(field.getModifiers()) && field.getType() == MapColor.class) {
                try {
                    field.setAccessible(true);
                    MapColor value = (MapColor)field.get(null);
                    colours.add(value.color);
                } catch (IllegalAccessException e) {
                    throw new RuntimeException(e);
                }
            }
        }

        byte[] bakedValues = new byte[256 * 256 * 256];
        float[] multipliers = {0.71f, 0.86f, 1f, 0.53f};

        for (int r = 0; r < 256; r++) {
            for (int g = 0; g < 256; g++) {
                for (int b = 0; b < 256; b++) {
                    float closestDist = Float.POSITIVE_INFINITY;
                    int closestIndex = 0;

                    for (int colourIndex = 4; colourIndex < colours.size(); colourIndex++) {
                        int colour = colours.get(colourIndex);
                        for (int multiplierIndex = 0; multiplierIndex < multipliers.length; multiplierIndex++) {
                            float multiplier = multipliers[multiplierIndex];

                            float red = (float)((colour >>> 16) & 255) * multiplier;
                            float green = (float)((colour >>> 8) & 255) * multiplier;
                            float blue = (float)(colour & 255) * multiplier;

                            float dist = ((float)r - red)*((float)r - red) + ((float)g - green)*((float)g - green) + ((float)b - blue)*((float)b - blue);
                            if (dist < closestDist) {
                                closestDist = dist;
                                closestIndex = colourIndex * 4 + multiplierIndex;
                            }
                        }
                    }

                    bakedValues[r << 16 | g << 8 | b] = (byte)closestIndex;
                }
            }
        }

        File bakedFile = new File(bakedFilePath);
        bakedFile.getParentFile().mkdirs();
        try {
            bakedFile.createNewFile();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        try {
            Files.write(Paths.get(bakedFilePath), bakedValues, StandardOpenOption.TRUNCATE_EXISTING);
            System.out.println("Wrote to file");
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        System.exit(0);
    }
}

package yes.mediumdifficulty.extractor.client;

import net.fabricmc.api.ClientModInitializer;
import net.minecraft.block.MapColor;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.List;

public class ExtractorClient implements ClientModInitializer {
    /**
     * Runs the mod initializer on the client environment.
     */
    @Override
    public void onInitializeClient() {
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

        StringBuilder mapColoursRs = new StringBuilder(String.format("""
                use valence::util::{vec3, Vec3};
                
                #[allow(clippy::approx_constant)]
                pub const MAP_COLOURS: [Vec3; %s] = [
                """, colours.size() * 4));

        for (int colour : colours) {
            mapColoursRs.append(
                    String.format(
                            """
                                    \tvec3(%s, %s, %s),
                                    \tvec3(%s, %s, %s),
                                    \tvec3(%s, %s, %s),
                                    \tvec3(%s, %s, %s),
                                    """,
                            ((float)(colour >>> 16) / 255f) * 0.71f,
                            ((float)((colour >>> 8) & 255) / 255f) * 0.71f,
                            ((float)(colour & 255) / 255f) * 0.71f,

                            ((float)(colour >>> 16) / 255f) * 0.86f,
                            ((float)((colour >>> 8) & 255) / 255f) * 0.86f,
                            ((float)(colour & 255) / 255f) * 0.86f,

                            ((float)(colour >>> 16) / 255f),
                            ((float)((colour >>> 8) & 255) / 255f),
                            ((float)(colour & 255) / 255f),

                            ((float)(colour >>> 16) / 255f) * 0.53f,
                            ((float)((colour >>> 8) & 255) / 255f) * 0.53f,
                            ((float)(colour & 255) / 255f) * 0.53f
                    )
            );
        }
        mapColoursRs.append("];");

        System.out.println(mapColoursRs);

        System.exit(0);
    }
}
